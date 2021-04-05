import numpy as np

from sklearn.linear_model import LogisticRegression, SGDClassifier
from sklearn.svm import SVC
from sklearn.pipeline import Pipeline
from sklearn.model_selection import RandomizedSearchCV

from utilities import check_correct


def logistic_cv(X_train, y_train):
    check_correct(X_train)

    # I've found that less regularization (i.e. a more flexible model)
    # works better for the review data.
    params = {"C": np.arange(1, 51)}

    # Elastic net regularization performs really well but takes forever to
    # train for more complex models. I'm trying to keep this model simpler
    # anyway.
    return RandomizedSearchCV(LogisticRegression(random_state=42,
                                                 solver="saga",
                                                 max_iter=2048),
                              params,
                              n_jobs=-1,
                              random_state=42).fit(X_train, y_train)


def support_vector_cv(X_train, y_train):
    check_correct(X_train)

    # Support vector machines take forever to fit with larger data sets.
    # Likely won't run this model with the full data set over something like
    # stochastic gradient descent.
    params = {"C": np.arange(1, 11)}

    return RandomizedSearchCV(SVC(cache_size=512, random_state=42),
                              params,
                              n_jobs=-1,
                              pre_dispatch=8,
                              random_state=42).fit(X_train, y_train)


def sgd_cv(X_train, y_train):
    check_correct(X_train)

    # https://scikit-learn.org/stable/modules/sgd.html#tips-on-practical-use
    params = {"penalty": ["l1", "l2", "elasticnet"],
              "alpha": 10.0**-np.arange(1, 7)}
    # This caused convergence warnings.
    # iterations = np.ceil(10**6/X_train.getnnz())

    return RandomizedSearchCV(SGDClassifier(max_iter=4096,
                                            random_state=42),
                              params,
                              n_jobs=-1,
                              pre_dispatch=8,
                              random_state=42).fit(X_train, y_train)
