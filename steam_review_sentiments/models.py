import numpy as np

from sklearn.feature_extraction.text import CountVectorizer, TfidfVectorizer
from sklearn.linear_model import LogisticRegression
from sklearn.svm import SVC
from sklearn.pipeline import Pipeline
from sklearn.model_selection import RandomizedSearchCV, train_test_split

from scipy.sparse.csr import csr_matrix


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
                                                    token.is_stop and stop)


def transform_string(doc, no_stop=True):
    """Transform a single string using spaCy."""
    return np.array([t.lemma_.lower().strip() for t in doc
                     if token_check(t, no_stop)])


def null_preproc(doc):
    """Do nothing."""
    return doc


def get_corpus(docs, no_stop=True):
    """Transform docs into an ndarray of tokens per doc.

    Parameters
    ----------
    docs : List[spacy.tokens.Doc]
        List of spaCy Docs.
    no_stop : bool, optional
        Whether to remove stop words. The default is False.

    Returns
    -------
    corpus : np.ndarray[np.ndarray]
        Documents.
    """
    corpus = np.empty(len(docs), np.ndarray)
    for idx, doc in enumerate(docs):
        corpus[idx] = transform_string(doc)

    return corpus


def split(docs, y):
    return train_test_split(docs, y, random_state=42, stratify=y)


def tfidf_transform(X_train, X_test):
    """Transforms and vectorizes the training then test sets.

    Train is transformed first followed by the test set using the same object.
    This is mostly a convenience function because trying multiple models
    with a Pipeline would refit the TfIdf (I think).

    Parameters
    ----------
    X_train : Iterable[spacy.tokens.Doc]
        Corpus.
    X_test : Iterable[spacy.tokens.Doc]
        Corpus.

    Returns
    -------
    X_train_tfidf : sparse.csr.csr_matrix
        Transformed X_train.
    X_test_tfidf : sparse.csr.csr_matrix
        Transform y_train.
    """
    tfidf = TfidfVectorizer(preprocessor=null_preproc,
                            tokenizer=transform_string,
                            token_pattern=None,
                            ngram_range=(1, 3))

    # Fit and transform training set followed by...
    X_train_tfidf = tfidf.fit_transform(X_train)
    # ONLY transforming the testing set.
    X_test_tfidf = tfidf.transform(X_test)

    return X_train_tfidf, X_test_tfidf


def logistic_cv(X_train, y_train):
    if not isinstance(X_train, csr_matrix):
        raise ValueError("Yeah, so, you probably passed in the dense matrix "
                         "which means that the model is going to eat up 10"
                         "GBs of RAM and have a terrible accuracy.\n\n"
                         "No one wants that. No one.")

    params = {"C": np.arange(0.001, 1.001, 0.01)}

    return RandomizedSearchCV(LogisticRegression(random_state=42,
                                                 solver="saga",
                                                 max_iter=2048),
                              params,
                              n_jobs=-1,
                              random_state=42).fit(X_train, y_train)


def support_vector_cv(X_train, y_train):
    if not isinstance(X_train, csr_matrix):
        raise ValueError("Yeah, so, you probably passed in the dense matrix "
                         "which means that the model is going to eat up 10"
                         "GBs of RAM and have a terrible accuracy.\n\n"
                         "No one wants that. No one.")
