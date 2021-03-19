import pandas as pd
import numpy as np
import spacy
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
    # Get unique titles to add to the EntityRuler patterns
    titles = steam_reviews.title.unique()

    # Throw an error if titles aren't lowercase.
    if np.apply_along_axis(np.frompyfunc(str.isupper, 1, 1), 0, titles):
        raise ValueError("All game titles should be lowercase.")

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
    if titles_iter:
        pass

    return doc


@Language.component("add_game_entity")
def add_game_entity(doc, matcher=None):
    # WORK_OF_ART
    if matcher:
        pass

    return doc


def load(path):
    steam_rev= pd.read_csv(path, low_memory=False)
    steam_rev.title = steam_rev.title.str.lower().str.strip()

    #
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
    nlp.add_pipe("add_title",
                 after="entity_ruler",
                 config={"titles_iter": steam_rev.title.iteritems()})

    # Reviews are short so I'm using a large batch size.
    return steam_rev, nlp, list(nlp.pipe(steam_rev.user_review,
                                         cleanup=True,
                                         batch_size=256,
                                         n_process=-1))

