from _liteparse import LiteParse

PARSER = LiteParse(ocr_enabled=False)


def parse(path: str) -> None:
    PARSER.parse(path)


PATH = "./dataset/1_page.pdf"


if __name__ == "__main__":
    parse(PATH)
