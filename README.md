# Attia

## About

This is a tool to search through [Peter
Attia's](https://peterattiamd.com/podcast/archive/) show notes, which are only
available to subscribers.

The way the tool works is it logs into the website, downloads all the show
notes, and then provides a simple command line query tool.

I built this because there are quite a few episodes (246 at the time of
writing), and sometimes I want to go back and check something I previously
heard. It's not always obvious where to go back to. Often there are multiple
episodes on the same topic, or sometimes the gem is hiding in an AMA.

The expected workflow is:

- Run the tool with some queries about what you were looking for
- It produces up to 10 matches
- Refine the search as needed
- Finally, open the page link for the episode you wanted to check on

## Usage

You need a WebDriver compatible process running to do the initial download. See
[jonhoo/fantoccini](https://github.com/jonhoo/fantoccini) for instructions. I
chose to download geckodriver from
[mozilla/geckodriver](https://github.com/mozilla/geckodriver/releases).

You then need to setup a config file with your username and password. The
easiest way to do this is run the program once, then edit the file at the
location it spits out. If you don't fill in a username and password, this tool
is useless. The show notes are a member only feature.

Finally, run the tool with `--download --query "max heart rate"`. The first time
it runs, it'll be slow. Later, you can omit `--download` to only search cached
show notes. You can include `--download` at a later time to update your notes,
which will only download new notes.

## Future work

- Remove the need to do the WebDriver stuff. A normal HTML client is probably
  sufficient. I mostly just tried it for fun.
- grep for TODO for more stuff
