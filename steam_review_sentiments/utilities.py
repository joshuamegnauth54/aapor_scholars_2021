import numpy as np
import spacy

from scipy.sparse.csr import csr_matrix
from sklearn.feature_extraction.text import CountVectorizer, TfidfVectorizer
from sklearn.model_selection import train_test_split
from sklearn.decomposition import LatentDirichletAllocation
from keras.preprocessing.sequence import pad_sequences


def check_correct(X):
    if not isinstance(X, csr_matrix):
        raise ValueError("Yeah, so, you probably passed in the dense matrix "
                         "which means that the model is going to eat up 10"
                         "GBs of RAM and have a terrible accuracy.\n\n"
                         "No one wants that. No one.")


def token_check(token, stop):
    """Remove stop words and punctuation if entity isn't "WORK_OF_ART".

    Parameters
    ----------
    token : spacy.tokens.Token
        SpaCy's Token class.
    stop : bool
        Remove stop words.

    Returns
    -------
    bool
        True if token shouldn't be dropped.
    """
    return token.ent_type_ == "WORK_OF_ART" or not (token.is_punct or
                                                    token.is_stop and stop or
                                                    not token.is_ascii)


def transform_string(doc, no_stop=True):
    """Transform a single string using spaCy."""
    # Not checking the types here because it'd fail with a reasonable
    # error message regardless.
    return np.array([t.lemma_.lower().strip() for t in doc
                     if token_check(t, no_stop)])


def transform_all(docs, no_stop=True):
    """Transform docs into an ndarray of lemmas per doc.

    Parameters
    ----------
    docs : List[spacy.tokens.Doc]
        List of spaCy Docs.
    no_stop : bool, optional
        Whether to remove stop words. The default is True.

    Returns
    -------
    corpus : np.ndarray[np.ndarray]
        Arrays of arrays of lemmas.
    """
    return np.array([transform_string(doc, no_stop) for doc in docs],
                    dtype=np.ndarray)


def tokenize_text(text, vectorizer, null_idx):
    """Tokenize text into ints via vectorizer's vocabulary.

    Parameters
    ----------
    text : Iterable[str]
        Text to tokenize as an array of str.
    vectorizer : CountVectorizer or TfidfVectorizer
        Text vectorizer from sklearn.feature_extraction.text.
    null_idx : uint
        Index representing word not present in vocabulary.

    Returns
    -------
    np.ndarray[np.uint32]
        Tokenized text.
    """
    return np.array([vectorizer.vocabulary_.get(word, null_idx)
                     for word in text],
                    dtype=np.uint32)


def tokenize_all(texts, vectorizer, null_idx, pad=True, max_text_len=None):
    if not isinstance(texts, (np.ndarray, list)):
        raise ValueError("Texts should be a nested array of strings.")
    if not isinstance(texts[0][0], str):
        raise ValueError("Texts should hold strings in each array.")

    # Tokenize each text
    texts_tokens = np.array([tokenize_text(text,
                                           vectorizer,
                                           null_idx) for text in texts],
                            dtype=np.ndarray)

    if pad:
        # Length of longest text.
        if not max_text_len:
            max_text_len = len(max(texts_tokens, key=len))

        # Pad text_tokens with null_idx.
        texts_tokens = pad_sequences(texts_tokens,
                                     maxlen=max_text_len,
                                     value=null_idx)

    return texts_tokens, max_text_len


def null_preproc(doc):
    """Do nothing."""
    return doc


def split(docs, y):
    """Split docs via y.

    This is a convenience wrapper around train_test_split.

    Parameters
    ----------
    docs : Iterable[Texts]
        Iterable of texts (strings or Docs).
    y : Iterable.
        Response to split and stratify on.

    Returns
    -------
    np.ndarray
        Four arrays of split data.
    """
    return train_test_split(docs, y, random_state=42, stratify=y)


def tfidf_transform(X_train, X_test, max_features=None):
    """Transform and vectorize the training set then test set.

    Train is transformed first followed by the test set using the same object.
    This is mostly a convenience function because trying multiple models
    with a Pipeline would refit the TfIdf (I think).

    Parameters
    ----------
    X_train : Iterable[spacy.tokens.Doc]
        Corpus.
    X_test : Iterable[spacy.tokens.Doc]
        Corpus.
    max_features : uint
        Maximum number of features. Passed down to TfidfVectorizer.

    Returns
    -------
    X_train_tfidf : sparse.csr.csr_matrix
        Transformed X_train.
    X_test_tfidf : sparse.csr.csr_matrix
        Transform y_train.
    """
    tfidf = TfidfVectorizer(strip_accents="unicode",
                            preprocessor=null_preproc,
                            tokenizer=transform_string,
                            token_pattern=None,
                            ngram_range=(1, 3),
                            max_features=max_features)

    # Fit and transform training set followed by...
    X_train_tfidf = tfidf.fit_transform(X_train)
    # ONLY transforming the testing set.
    X_test_tfidf = tfidf.transform(X_test)

    return X_train_tfidf, X_test_tfidf, tfidf


def predict(nlp, vectorizer, model, new_data):
    """Transform new_data and predict using model.

    Parameters
    ----------
    nlp : spacy.lang.en.English
        SpaCy language object.
    vectorizer : sklearn.feature_extraction.text.TfidfVectorizer
        Fit TfidfVectorizer or CountVectorizer.
    model : Fit sklearn model class.
        Any fitted sklearn model or equivalent.
    new_data : np.ndarray[Doc]
        NumPy array or List of Docs.

    Raises
    ------
    ValueError
        Invalid new_data (i.e. not a sequence).

    Returns
    -------
    np.ndarray
        Predictions.
    """
    if not isinstance(new_data, (list, np.ndarray)):
        raise ValueError("The new_data parameter should be a list.")

    # Process the data with our spaCy Language object.
    X_new = np.array([nlp(data) for data in new_data])
    # And transform with the Tf-Idf fit on the training data.
    X_new = vectorizer.transform(X_new)

    return model.predict(X_new)


def topic_modeling(docs, max_features=None, max_topics=10, top_topics=10):

    if not isinstance(docs, (list, np.ndarray)):
        raise ValueError("The docs parameter should be a list.")
    if not isinstance(docs[0], spacy.tokens.Doc):
        raise ValueError("The docs parameter should contain spaCy Docs.")

    # CountVectorizer is used as my BoW model here despite Gensim having
    # more robust utilities. The reason? Laziness.
    # I'm using a higher min_df here since I'm not really building a model.
    vectorizer = CountVectorizer(strip_accents="unicode",
                                 preprocessor=null_preproc,
                                 tokenizer=transform_string,
                                 token_pattern=None,
                                 ngram_range=(1, 3),
                                 min_df=2,
                                 max_features=max_features)

    # Transform into sparse array
    docs_mat = vectorizer.fit_transform(docs)

    # Finally, fit the model and return some topics!
    lda = LatentDirichletAllocation(n_components=max_topics,
                                    n_jobs=-1,
                                    random_state=42).fit(docs_mat)

    # Can't think of a better way to do this.
    # The features are stored in a List but converting the List to an ndarray
    # leads to a massive consumption of memory.
    # I'm sure get_feature_names() isn't returning a copy each time, right?
    features = vectorizer.get_feature_names()
    topics = np.empty(10, np.ndarray)

    for idx, component in enumerate(lda.components_):
        # Sort and get top_topics indices
        indices = component.argsort()[-top_topics:]
        # (See above). Features is a List so I can't use fancy indexing.
        topics[idx] = np.array([features[i] for i in indices])

    return topics
