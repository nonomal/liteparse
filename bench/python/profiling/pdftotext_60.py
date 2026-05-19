import pdftotext


def parse(path: str) -> None:
    with open(path, "rb") as f:
        pdf = pdftotext.PDF(f)
    "\n\n".join(pdf)


PATH = "./dataset/60_pages.pdf"


if __name__ == "__main__":
    parse(PATH)
