# Changelog

Currently, the changelog is built locally. It will be moved to CI once labels stabilize.

For now, a bit of preparation is required before you can run the script:
- fetch the srtool digests
- store them under the `digests` folder as `<chain>-srtool-digest.json`
- ensure the `.env` file is up to date with correct information

The content of the release notes is generated from the template files under the `scripts/changelog/templates` folder. For readability and maintenance, the template is split into several small snippets.

Run:
```
./changelog.sh <ref_since> [<ref_until>=HEAD]
```

For instance:
```
./changelog.sh statemine-v5.0.0
```

A file called `release-notes-cumulus.md` will be generated and can be used for the release.

## Considered labels

The following list will likely evolve over time and it will be hard to keep it in sync.
In any case, if you want to find all the labels that are used, search for `meta` in the templates.
Currently, the considered labels are:

- Priority: C<N> labels
- Audit: D<N> labels
- E4 => new host function
- B0 => silent, not showing up
- B1-releasenotes (misc unless other labels)
- B7-runtimenoteworthy (runtime changes)
- B5-client (client changes)
- T6-XCM

Note that labels with the same letter are mutually exclusive.
A PR should not have both `B0` and `B5`, or both `C1` and `C9`. In case of conflicts, the template will
decide which label will be considered.

## Dev and debuggin

### Hot Reload

The following command allows **Hot Reload**:
```
fswatch templates -e ".*\.md$" | xargs -n1 -I{} ./changelog.sh statemine-v5.0.0
```
### Caching

By default, if the changelog data from Github is already present, the calls to the Github API will be skipped
and the local version of the data will be used. This is much faster.
If you know that some labels have changed in Github, you probably want to refresh the data.
You can then either delete manually the `cumulus.json` file or `export NO_CACHE=1` to force refreshing the data.
