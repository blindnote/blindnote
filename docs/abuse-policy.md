# BlindNote abuse policy

BlindNote is a one-time-read, end-to-end encrypted note service. The operator **cannot read
note contents** — notes are encrypted in the sender's browser with a key that never reaches
the server. This shapes everything below: we cannot moderate what we cannot see, so abuse
handling is built to work *without* reading notes.

## Reporting abuse

Email **ye@deligoz.me** with the BlindNote link you are reporting. This address matches the
domain's WHOIS contact and is monitored.

## How we act on a report

- **Report by link.** Anyone holding a link can report it. The server burns the note by its
  id **without decrypting anything** — the same action a reader would trigger, so the note
  becomes unavailable and its key is destroyed. This is the only takedown mechanism that does
  not break the zero-knowledge guarantee, and it is enough: the reported link stops working.
- We do not, and technically cannot, hand over note plaintext, because we never have it.
- We keep no access log and do not log note ids or request bodies, so we cannot produce a
  history of who created or read a note.

## Why abuse is structurally limited here

- **Notes are never rendered as HTML and never auto-linkified.** A service that produces no
  clickable links is of little use for phishing.
- **Notes self-destruct** — a 24-hour maximum lifetime in the current version — which narrows
  any abuse window to hours, not forever.
- **Per-IP create quotas** sit on top of the general rate limit, so the service cannot be used
  as a bulk-messaging or spam relay.

## For hosting and infrastructure providers

If you have received an abuse complaint about content hosted via BlindNote: the operator is
responsive at the address above and can make any reported link stop working immediately,
without needing (or being able) to read it. We ask for the chance to act on a specific report
before broader action is taken, because a suspension takes down **every** pending note,
including unread ones belonging to uninvolved people.

_Last updated: 2026-10-29._
