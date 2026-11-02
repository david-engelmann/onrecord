"""Canonical form 1 digests, consumed pinned (OR-6a).

Citations name a tape by its digest under flighttape
canonical form 1, the SHA-256 of the canonical form defined
by the flighttape contract obligations FT-26 to FT-28:
object keys sorted by UTF-16 code units, numbers rendered
as ECMAScript renders them, no whitespace. This module
implements that form from its specification so onrecord
can derive claim ids from content and position. It is a
consumption of the pinned format, never a fork of it: the
test suite pins the digest of a known tape to the golden
value the flighttape programme published for it, so any
drift from the form fails loudly here.
"""
from __future__ import annotations

import hashlib
import json
import math


class DigestError(ValueError):
    """A value that cannot be canonicalized."""


def _render_number(value):
    """Render a finite float as ECMAScript renders it.

    repr gives the shortest digits that round trip. From
    them, point is the count of digits before the decimal
    point when the value is written out in decimal, and the
    four cases at the end are the ECMAScript number to
    string cases.
    """
    if value == 0:
        return "0"
    sign = "-" if value < 0 else ""
    text = repr(abs(value))
    if "e" in text:
        mantissa, _, exponent = text.partition("e")
        exp10 = int(exponent)
    else:
        mantissa, exp10 = text, None
    if "." in mantissa:
        whole, _, fraction = mantissa.partition(".")
    else:
        whole, fraction = mantissa, ""
    raw = whole + fraction
    digits = raw.lstrip("0")
    if exp10 is None:
        leading = len(raw) - len(digits)
        exp10 = len(whole) - leading - 1
    digits = digits.rstrip("0") or "0"
    point = exp10 + 1
    count = len(digits)
    if count <= point <= 21:
        return sign + digits + "0" * (point - count)
    if 0 < point <= 21:
        return sign + digits[:point] + "." + digits[point:]
    if -6 < point <= 0:
        return sign + "0." + "0" * (-point) + digits
    head = digits if count == 1 else digits[0] + "." + digits[1:]
    return (sign + head + "e" + ("+" if exp10 >= 0 else "-")
            + str(abs(exp10)))


def _render(value, out):
    if value is None:
        out.append("null")
    elif value is True:
        out.append("true")
    elif value is False:
        out.append("false")
    elif isinstance(value, str):
        out.append(json.dumps(value, ensure_ascii=False))
    elif isinstance(value, int):
        out.append(str(value))
    elif isinstance(value, float):
        if not math.isfinite(value):
            raise DigestError("the value holds a non-finite number")
        out.append(_render_number(value))
    elif isinstance(value, list):
        out.append("[")
        for index, item in enumerate(value):
            if index:
                out.append(",")
            _render(item, out)
        out.append("]")
    elif isinstance(value, dict):
        keys = list(value)
        for key in keys:
            if not isinstance(key, str):
                raise DigestError(
                    "the value holds an object key that is not a string")
        out.append("{")
        ordered = sorted(keys, key=lambda k: k.encode("utf-16-be"))
        for index, key in enumerate(ordered):
            if index:
                out.append(",")
            out.append(json.dumps(key, ensure_ascii=False))
            out.append(":")
            _render(value[key], out)
        out.append("}")
    else:
        raise DigestError(
            "the value holds an unsupported type, "
            f"{type(value).__name__}")


def canonical_bytes(value):
    """The value's canonical form 1 as UTF-8 bytes."""
    out = []
    try:
        _render(value, out)
        return "".join(out).encode("utf-8")
    except DigestError:
        raise
    except RecursionError:
        raise DigestError(
            "the value is nested beyond the interpreter limit") from None
    except UnicodeEncodeError:
        raise DigestError(
            "the value holds text that cannot be encoded") from None
    except ValueError:
        raise DigestError(
            "the value holds a value that cannot be rendered") from None


def digest_of(value):
    """The SHA-256 hex digest of a value under canonical form 1."""
    return hashlib.sha256(canonical_bytes(value)).hexdigest()
