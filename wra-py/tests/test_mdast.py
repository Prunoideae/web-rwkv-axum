from mistune import create_markdown
from os import path
from pprint import pprint

markdown = create_markdown(renderer=None)

test_md = path.join(path.dirname(__file__), "test.md")

result = markdown(open(test_md).read())
pprint(result)
