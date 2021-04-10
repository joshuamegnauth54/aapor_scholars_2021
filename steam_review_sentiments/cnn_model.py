import numpy as np
import keras
from sklearn.feature_extraction.text import CountVectorizer
from keras.models import Sequential
from keras.layers import Conv1D, Dense, Embedding, Flatten
from keras.layers.pooling import GlobalMaxPooling1D

from utilities import null_preproc, transform_string


def get_embeddings(nlp, X_train, X_test):

    vectorizer = CountVectorizer(strip_accents="unicode",
                                 preprocessor=null_preproc,
                                 tokenizer=transform_string,
                                 token_pattern=None).fit(X_train)
    # The vocabulary size only consists of the terms that appear after
    # Tf-Idf. This is our first dimension.
    vocab_size = len(vectorizer.get_feature_names())
    # Length of the pre-trained word embeddings vector
    vec_size = nlp.vocab.vectors_length
    # Finally, initialize a zero length ndarray with the sizes.
    embeddings = np.zeros((vocab_size, vec_size), dtype=np.float32)

    # CountVectorizer.vocabulary_ is a dictionary matching word to index.
    # Thus:
    # index = vectorizer.vocabulary_["meow"]
    # value = vectorizer.get_feature_names()[index]
    # value == "meow"
    for word, i in vectorizer.vocabulary_.items():
        # print(f"word: {word}, index: {i}")
        embeddings[i] = nlp.vocab[str(word)].vector

    return embeddings