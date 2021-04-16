import numpy as np
import keras
from sklearn.feature_extraction.text import CountVectorizer
from keras.models import Sequential
from keras.layers import BatchNormalization, Conv1D, Dense, Embedding
from keras.layers.pooling import GlobalMaxPooling1D
from keras.initializers import Constant

from utilities import null_preproc, transform_string,\
    transform_all, tokenize_all


def get_embeddings(nlp, X_train, X_test):
    """Return word vectors and embeddings for X_train and X_test.

    The returned vectors are a subset of the spaCy language object's vectors
    that only include words present in X_train.

    X_train and X_test should be Docs rather than the TF-IDF transformed
    sparse arrays.

    Parameters
    ----------
    nlp : spacy.lang.en.English
        Trained spaCy language object.
    X_train : np.ndarray[spacy.tokens.Doc]
        Array of spaCy Docs (training).
    X_test : np.ndarray[spacy.tokens.Doc]
        Array of spaCy Docs (testing).

    Returns
    -------
    embeddings : np.ndarray[np.float32]
        Subsetted word embeddings.
    X_train_tokens : np.ndarray[np.int32]
        Word embeddings for X_train.
    X_test_tokens : np.ndarray[np.int32]
        Word embeddings for X_test.
    """

    # CountVectorizer counts each word/token, so I can use it to extract
    # ONLY the vectors present in my data from spaCy's pretrained embeddings.
    vectorizer = CountVectorizer(strip_accents="unicode",
                                 preprocessor=null_preproc,
                                 tokenizer=transform_string,
                                 token_pattern=None).fit(X_train)
    # The vocabulary size only consists of the terms that appear after
    # vectorizing. This is our first dimension.
    # 0 will be used as an indicator for missing words, so let's shift the
    # vocab by elements + 1.
    vocab_size = len(vectorizer.get_feature_names()) + 1
    # Length of the pre-trained word embeddings vector (300 most likely)
    vec_size = nlp.vocab.vectors_length
    # Finally, initialize a zero length ndarray with the sizes.
    embeddings = np.zeros((vocab_size, vec_size), dtype=np.float32)

    # CountVectorizer.vocabulary_ is a dictionary matching word to index.
    # Thus:
    # index = vectorizer.vocabulary_["meow"]
    # value = vectorizer.get_feature_names()[index]
    # value == "meow"
    for word, i in vectorizer.vocabulary_.items():
        # Can't index with NumPy strings. Also, shift the embeddings by 1.
        embeddings[i + 1] = nlp.vocab[str(word)].vector

    # Tokenize the training and test sets. 0 is the magic NaN value.
    X_train_tokens = tokenize_all(transform_all(X_train),
                                  vectorizer,
                                  0,
                                  True)

    X_test_tokens = tokenize_all(transform_all(X_test),
                                 vectorizer,
                                 0,
                                 True)

    return embeddings, X_train_tokens, X_test_tokens


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
