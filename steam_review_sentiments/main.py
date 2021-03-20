from preprocessing import load, add_title_bad

if __name__ == '__main__':
    steam_reviews, nlp, docs = load(r"steam_reviews.csv")
    add_title_bad(docs, steam_reviews)
