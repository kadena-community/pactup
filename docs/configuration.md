# Configuration

pactup comes with many features out of the box. Some of them are not activated by default as they’re changing your shell default behavior, and some are just a feature flag to avoid breaking changes or just experimental until we decide it is worthwhile to introduce them.

All these features can be configured by adding flags to the `pactup env` call when initializing the shell. For instance, if your shell set up looks like `eval "$(pactup env)"` then you can add a flag to it by changing it to `eval "$(pactup env --my-flag=value)"`

Here’s a list of these features and capabilities:

### `--use-on-cd`

**✅ Highly recommended**

`--use-on-cd` appends output to `pactup env`'s output that will hook into your shell upon changing directories, and will switch the Pact version based on the requirements of the current directory, based on `.pact-version` or `.pactrc`.

This allows you do avoid thinking about `pactup use`, and only `cd <DIR>` to make it work.

### `--version-file-strategy=recursive`

**✅ Highly recommended**

Makes `pactup use` and `pactup install` take parent directories into account when looking for a version file ("dotfile")--when no argument was given.

So, let's say we have the following directory structure:

```
repo/
├── package.json
├── .pact-version <- with content: `4.13.0`
└── packages/
  └── my-package/ <- I am here
    └── package.json
```

And I'm running the following command:

```sh-session
repo/packages/my-package$ pactup use
```

Then pactup will switch to Pact v4.13.0

Without the explicit flag, the value is set to `local`, which will not traverse the directory tree and therefore will print:

```sh-session
repo/packages/my-package$ pactup use
error: Can't find version in dotfiles. Please provide a version manually to the command.
```
