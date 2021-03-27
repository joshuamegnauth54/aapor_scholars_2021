import pandas as pd
import numpy as np
import spacy
import json

from spacy.tokens import Doc, Span, Token
from spacy.matcher import Matcher
from spacy.language import Language
# from spacy.lang.en import STOP_WORDS


def add_titles_ent_pipe(steam_reviews, nlp):
    """Add pipeline to nlp to set game titles as WORK_OF_ART.

    Parameters
    ----------
    steam_reviews : pandas.DataFrame
        DataFrame of Steam reviews with preprocessed titles.
    nlp : spacy.lang.en.English
        Spacy English language object.

    Raises
    ------
    ValueError
        Raises on unpreprocessed titles.

    Returns
    -------
    None.

    """
    # Get unique, lowercase titles to add to the EntityRuler patterns
    titles = steam_reviews.title.str.lower().str.strip().unique()

    # Add in acronyms and whatnot. I'll load this from a file later.
    # Also, this is incomplete. I'm adding these as a notice them.
    # Totally scientific by the way.
    # I'm avoiding really common words. But there has to be a better way
    # to do this (topic modeling?).
    titles = np.concatenate([titles, ["dota", "wc3", "nwn", "eso",
                                      "ygo", "poe", "skyrim", "oblivion",
                                      "new vegas", "fallout", "fallout nv",
                                      "morrowind", "bf1", "bf2", "bf3",
                                      "bad company", "bfbc"]])

    # Throw an error if titles aren't lowercase.
    # if any(np.apply_along_axis(np.frompyfunc(str.isupper, 1, 1), 0, titles)):
    #    raise ValueError("All game titles should be lowercase.")

    # Titles are REALLY short so 256 is okay as a batch size.
    # The pipe function returns a generator of Doc.
    # For each Token (title_piece) build a pattern dict to match the title
    # where punctuation (i.e. colons or whatever) are optional.
    # The "label" key is for EntityRuler.
    # Is this write once read once? A for loop seems equally messy.
    patterns = [{"label": "WORK_OF_ART",
                 "pattern": [{"LOWER": title_piece.text}
                             if not title_piece.is_punct
                             else {"TEXT": title_piece.text, "OP": '?'}
                             for title_piece in doc]}
                for doc in nlp.pipe(titles,
                                    cleanup=True,
                                    batch_size=256,
                                    n_process=-1)]

    # I'm not sure about phrase_matcher_attr or validate.
    # https://spacy.io/api/entityruler
    # But I'm presuming they're good to use.
    # I'm overwriting entities after NER (Named Entity Recognition) to
    # replace the model's (likely incorrect) labels.
    entity_ruler = nlp.add_pipe("entity_ruler",
                                after="ner",
                                config={"phrase_matcher_attr": "LOWER",
                                        "validate": True,
                                        "overwrite_ents": True
                                        })

    # Finally, add the patterns
    entity_ruler.add_patterns(patterns)


@Language.component("add_title")
def add_title(doc, titles_iter=None):
    """spaCy component to add video game titles to each document.

    Parameters
    ----------
    doc : spacy.tokens.Doc
        Document passed to component (i.e. by a pipeline).
    titles_iter : Iterable[Tuple[Union[Hashable, NoneType], Any]], optional
        Iterator of (index, title) tuples. The iterator should be the length
        of the DataFrame.

    Raises
    ------
    StopIteration
        Raises if titles_iter isn't the same length as the piped in docs.

    Returns
    -------
    doc : spacy.tokens.Doc
        Same doc that was passed in (because you know...pipeline).
    """
    if titles_iter:
        doc._.title = next(titles_iter)

    return doc


def add_title_bad(docs, steam_reviews):
    """Add game title to Doc[s] inefficiently.

    Parameters
    ----------
    docs : Iterable[spacy.tokens.Doc]
        Iterator of spaCy Docs.
    steam_reviews : pd.DataFrame
        DataFrame of Steam review data.

    Returns
    -------
    None.

    """
    for doc, title in zip(docs, steam_reviews.title.items()):
        doc._.title = title[1]


def normalize_words(reviews):
    """To do: Function to eat words and digest them into useful bits.
    Check some common acronyms to turn into titles.
    Replace terms like tps or fps by shooter.
    """

    # Replace long sequences of curse filter avoidance with #*%@.
    # I mean...you know what I mean.
    # Note that the string of ****** may be something entirely different,
    # but I've noticed they're expletives more often than not.
    curse_pat = r"([♥|*|@|$|%])\1{3,}"
    reviews.user_review = reviews.user_review.str.replace(curse_pat,
                                                          "fuck",
                                                          regex=True)

    with open("cleanup.json", 'r') as cleanup_file:
        cleanup = json.load(cleanup_file)

        # gen = {pair["term"]: pair["replace"] for pair in cleanup["generic"]}
        # reviews.user_review.replace(gen, inplace=True)
        # I can't use Series.replace because I need case insensitive searching
        for rep in cleanup["generic"]:
            reviews.user_review = reviews.user_review.str.replace(rep["term"],
                                                                  rep["replace"],
                                                                  case=False)

        # Games (or series) may their own acronyms or jargon to replace.
        # This is woefully incomplete.
        for d in cleanup["game"]:
            mask = reviews.title.str.contains(d["title"],
                                              case=False)

            for term, rep in zip(d["term"], d["replace"]):
                reviews.loc[mask, "user_review"] =\
                    reviews[mask].user_review.str.replace(term,
                                                          rep,
                                                          case=False)


def load(path):
    """Load the data, spaCy model, and documents.

    Parameters
    ----------
    path : str
        Path to Steam review data CSV.

    Returns
    -------
    pd.DataFrame
        Review data.
    spacy.lang.en.English
        Language object with extra pipeline components.
    List[spacy.tokens.Doc]
        Documents passed through the NLP pipeline.
    """
    steam_rev = pd.read_csv(path, low_memory=False)
    normalize_words(steam_rev)
    # Steam reviews can be voted funny or given a thumbs up.
    # So...let's use these as our classes!
    steam_rev["up_funny"] = steam_rev.votes_up > steam_rev.votes_funny

    # Start with a pretrained model on blogs (maybe this is a bad idea?)
    nlp = spacy.load("en_core_web_md")

    # Store the title of each game and whether the review is positive/neg
    if not Doc.get_extension("is_recommended"):
        Doc.set_extension("is_recommended", default=np.nan)
    if not Doc.get_extension("title"):
        Doc.set_extension("title", default=np.nan)

    # Add extra pipeline components
    add_titles_ent_pipe(steam_rev, nlp)
    # Can't get this to work.
    # nlp.add_pipe("add_title",
    #             after="entity_ruler",
    #             config={"titles_iter": steam_rev.title.iteritems()})

    # Reviews are short so I'm using a large batch size.
    return steam_rev, nlp, np.array(list(nlp.pipe(steam_rev.user_review,
                                                  cleanup=True,
                                                  batch_size=256,
                                                  n_process=-1)),
                                    dtype=Doc)
