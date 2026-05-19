import pymupdf


def parse(path: str) -> None:
    with pymupdf.open(path) as doc:
        "\n\n".join(page.get_text() for page in doc)


PATH = "./dataset/250_pages.pdf"


if __name__ == "__main__":
    parse(PATH)
