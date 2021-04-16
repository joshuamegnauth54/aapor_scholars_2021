from preprocessing import load, add_title_bad
from utilities import split, tfidf_transform

if __name__ == '__main__':
    steam_reviews, nlp, docs = load(r"steam_reviews.csv")
    add_title_bad(docs, steam_reviews)

    # Split on user_suggestion
    X_train, X_test, y_train, y_test = split(docs,
                                             steam_reviews.user_suggestion)

    # TF-IDF and LDA
    X_train_tfidf, X_test_tfidf, tfidf = tfidf_transform(X_train, X_test)

