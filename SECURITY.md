# Security policy

MemoryStick opens documents that come from anywhere and renders them, HTML
included, in a web view. A document is untrusted input, so reports are taken
seriously and handled first.

## Supported versions

Only the latest release receives security fixes. Colony installs updates as
they are published, so staying current is one update away.

## Reporting a vulnerability

Report it privately through
[GitHub Security Advisories](https://github.com/Project-Colony/MemoryStick/security/advisories/new)
("Report a vulnerability"). Please do not open a public issue for something
that can be exploited.

What to expect:

- an acknowledgement within a few days;
- a fix, or a plan to mitigate it, before anything is made public, agreed
  with you; a fixed release usually ships as soon as the fix is merged;
- credit in the release notes, if you want it.

## What is in scope

Of particular interest is anything that lets a document:

- run script, or call MemoryStick's commands;
- read a file the user did not open, or make MemoryStick reach the network;
- navigate the window away from MemoryStick's own page, or open something
  other than a web or mail link outside the app.

Also in scope: the release chain, meaning anything that would let a build
reach users without the Project-Colony organisation's signature.

Out of scope: bugs in the operating system's web view (WebView2, WebKit,
WebKitGTK), which belong to its vendor, and in the libraries under
`dist/vendor/`, which belong to their projects. Do tell us if a MemoryStick
release ships a vulnerable version of one of them.
