from random import randint


def get_random(dup: set[str]) -> str:
    rand = str(randint(0, 2**31))
    while rand in dup:
        rand = str(randint(0, 2**31))
    return rand
