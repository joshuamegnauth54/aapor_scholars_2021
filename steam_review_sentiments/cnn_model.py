import numpy as np
import keras
import spacy
from sklearn.feature_extraction.text import CountVectorizer
from sklearn.exceptions import NotFittedError
from keras.models import Sequential
from keras.layers import BatchNormalization, Conv1D, Dense, Embedding
from keras.layers.pooling import GlobalMaxPooling1D
from keras.initializers import Constant

from utilities import null_preproc, transform_string,\
    transform_all, tokenize_all

# This class is badly designed. I wanted to leverage spaCy, but I combined
# tools in a very poor way...
class PadCounts:
    def __init__(self, nlp, pad_length=None):
        """Instantiate PadCounts.

        Parameters
        ----------
        nlp : spacy.lang.en.English
            Trained spaCy language object.
        pad_length : int, optional
            Set a predefined length to pad data in transform(). Calculated
            from X_train during fit() if None.

        Returns
        -------
        None.

        """
        # Language object for embeddings.
        self.__nlp = nlp
        # Word embeddings array.
        self.__embeddings = None
        # Sklearn model for word counts.
        self.__vectorizer = None
        # Vocabulary size based on X_train.
        self.__vocab_size = None
        # Length of the pre-trained word embeddings vector (300 most likely)
        self.__vec_size = None
        # Max length of a training document (or a predefined max for padding)
        self.__pad_length = pad_length

    def __to_docs(self, X):
        # Convert X to a list of Doc if necessary
        if isinstance(X[0], str):
            return np.array([self.__nlp(text) for text in X])
        else:
            return X

    def fit(self, X_train):
        """Fit PadCounts on X_train and transform into embeddings.

        Parameters
        ----------
        X_train : np.ndarray[spacy.tokens.Doc or str]
            Array of spaCy Docs or strings (training).

        Raises
        ------
        ValueError
            Raised if X_train isn't an array of spaCy Docs.

        Returns
        -------
        None.

        """
        if not isinstance(X_train, (np.ndarray, list)) or not len(X_train):
            raise ValueError("X_train needs to be an array of strs or Docs.")

        # Make sure X_train are Docs.
        X_train = self.__to_docs(X_train)

        # CountVectorizer counts each word/token, so I can use it to extract
        # ONLY the vectors present in my data from spaCy's pretrained
        # embeddings.
        self.__vectorizer = CountVectorizer(strip_accents="unicode",
                                            preprocessor=null_preproc,
                                            tokenizer=transform_string,
                                            token_pattern=None).fit(X_train)

        # The vocabulary size only consists of the terms that appear after
        # vectorizing. This is our first dimension.
        # 0 will be used as an indicator for missing words, so let's shift the
        # vocab by elements + 1.
        self.__vocab_size = len(self.__vectorizer.get_feature_names()) + 1
        # Word vectors length (second dimension).
        self.__vec_size = self.__nlp.vocab.vectors_length

        # Remove stop words, et cetera.
        # And yeah, due to bad design I'm calling transform_string a lot.
        X_transformed = transform_all(X_train)

        if not self.__pad_length:
            self.__pad_length = len(max(X_transformed, key=len))

    def embeddings(self):
        """Return subsetted embeddings for X_train.

        The returned vectors are a subset of the spaCy language object's
        vectors that only include words present in X_train.

        PadCounts should be fit() before calling embeddings().

        Raises
        ------
        NotFittedError
            Raised if PadCounts() is unfit.

        Returns
        -------
        embeddings : np.ndarray[np.float32]
            Subsetted word embeddings.
        """
        if self.__embeddings:
            return self.__embeddings
        elif not self.__vectorizer:
            raise NotFittedError("Call PadCounts.fit() first.")

        # Initialize a zero length ndarray with the vocab and vector sizes.
        self.__embeddings = np.zeros((self.__vocab_size, self.__vec_size),
                                     dtype=np.float32)

        # CountVectorizer.vocabulary_ is a dictionary matching word to index.
        # Thus:
        # index = vectorizer.vocabulary_["meow"]
        # value = vectorizer.get_feature_names()[index]
        # value == "meow"
        for word, i in self.__vectorizer.vocabulary_.items():
            # Can't index with NumPy strings.
            # Also, shift the embeddings by 1.
            self.__embeddings[i + 1] = self.__nlp.vocab[str(word)].vector


    def transform(self, X, remove_junk=True):
        """Return tokenized X.

        Parameters
        ----------
        X : np.ndarray[Doc or str]
            Array of Docs or str to tokenize.
        remove_junk : bool, optional
            Whether X needs to be transformed to remove stop words.
            The default is True.

        Raises
        ------
        NotFittedError
            DESCRIPTION.
        ValueError
            DESCRIPTION.

        Returns
        -------
        X_tokens : np.ndarray[np.int32]
            Word embeddings for X.
        """
        if not self.__vectorizer or not self.__pad_length:
            raise NotFittedError("Call PadCounts.fit() first.")
        if not isinstance(X, (np.ndarray, list)) or not len(X):
            raise ValueError("X_train needs to be an array of strs or Docs.")

        # Make sure X is a list of Docs
        X = self.__to_docs(X)

        # Remove stop words et cetera if necessary.
        if remove_junk:
            X = transform_all(X)

        # Tokenize the training and test sets. 0 is the magic NaN value.
        return tokenize_all(X,
                            self.__vectorizer,
                            0,
                            True,
                            self.__pad_length)


def cnn_model(embeddings, max_length, ngrams=3, dropout_prob=.4):

    # Base model. Convert to class later(?!?).
    model = Sequential(name="cnn_steam_reviews_model")

    # Embedding layer to use our pretrained vectors.
    # https://keras.io/examples/nlp/pretrained_word_embeddings/
    model.add(Embedding(embeddings.shape[0],
                        embeddings.shape[1],
                        embeddings_initializer=Constant(embeddings),
                        # mask_zero=True,
                        input_length=max_length,
                        trainable=False))

    # One dimension convulution layer
    model.add(Conv1D(max_length,
                     ngrams,
                     padding="same"))

    # Normalize inputs.
    model.add(BatchNormalization())

    # Max pooling
    model.add(GlobalMaxPooling1D())

    # Non-linearity and weight optimization
    model.add(Dense(128, activation="relu"))

    # Output
    model.add(BatchNormalization())
    model.add(Dense(1, activation="sigmoid"))

    # Compile and return
    model.compile("adam",
                  "binary_crossentropy",
                  ["accuracy"])

    return model

def model_def_fit(model, X_train, y_train, epochs):
    return model.fit(X_train,
                     y_train,
                     batch_size=128,
                     epochs=epochs,
                     workers=6,
                     use_multiprocessing=True,
                     validation_split=.25)
