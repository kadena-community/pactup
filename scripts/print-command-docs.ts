#!/usr/bin/env -S npx tsx

import cmd from "cmd-ts";
import cmdFs from "cmd-ts/dist/cjs/batteries/fs.js";
import { execa } from "execa";
import fs from "node:fs";
import { Writable } from "node:stream";

const PactupBinaryPath = {
  ...cmdFs.ExistingPath,
  defaultValue() {
    const target = new URL("../target/debug/pactup", import.meta.url);
    if (!fs.existsSync(target)) {
      throw new Error(
        "Can't find debug target, please run `cargo build` or provide a specific binary path"
      );
    }
    return target.pathname;
  },
};

const command = cmd.command({
  name: "print-command-docs",
  description: "prints the docs/command.md file with updated contents",
  args: {
    checkForDirty: cmd.flag({
      long: "check",
      description: `Check that file was not changed`,
    }),
    pactupPath: cmd.option({
      long: "binary-path",
      description: "the pactup binary path",
      type: PactupBinaryPath,
    }),
  },
  async handler({ checkForDirty, pactupPath }) {
    const targetFile = new URL("../docs/command.md", import.meta.url).pathname;
    await main(targetFile, pactupPath);
    if (checkForDirty) {
      const gitStatus = await checkGitStatus(targetFile);
      if (gitStatus.state === "dirty") {
        process.exitCode = 1;
        console.error(
          "The file has changed. Please re-run `pnpm generate-command-docs`."
        );
        console.error(`hint: The following diff was found:`);
        console.error();
        console.error(gitStatus.diff);
      }
    }
  },
});

cmd.run(cmd.binary(command), process.argv).catch((err) => {
  console.error(err);
  process.exitCode = process.exitCode || 1;
});

async function main(targetFile: string, pactupPath: string): Promise<void> {
  const stream = fs.createWriteStream(targetFile);

  const { subcommands, text: mainText } = await getCommandHelp(pactupPath);

  await write(stream, line(`pactup`, mainText));

  for (const subcommand of subcommands) {
    const { text: subcommandText } = await getCommandHelp(
      pactupPath,
      subcommand
    );
    await write(stream, "\n" + line(`pactup ${subcommand}`, subcommandText));
  }

  stream.close();

  await execa(`pnpm`, ["prettier", "--write", targetFile]);
}

function write(stream: Writable, content: string): Promise<void> {
  return new Promise((resolve, reject) => {
    stream.write(content, (err) => (err ? reject(err) : resolve()));
  });
}

function line(cmd: string, text: string): string {
  const cmdCode = "`" + cmd + "`";
  const textCode = "```\n" + text + "\n```";
  return `# ${cmdCode}\n${textCode}`;
}

async function getCommandHelp(
  pactupPath: string,
  command?: string
): Promise<{ subcommands: string[]; text: string }> {
  const cmdArg = command ? [command] : [];
  const result = await run(pactupPath, [...cmdArg, "--help"]);
  const text = result.stdout;
  const rows = text.split("\n");
  const headerIndex = rows.findIndex((x) => x.includes("Commands:"));
  const subcommands: string[] = [];
  if (!command) {
    for (const row of rows.slice(
      headerIndex + 1,
      rows.indexOf("", headerIndex + 1)
    )) {
      const [, word] = row.split(/\s+/);
      if (word && word[0].toLowerCase() === word[0]) {
        subcommands.push(word);
      }
    }
  }
  return {
    subcommands,
    text,
  };
}

function run(pactupPath: string, args: string[]) {
  return execa(pactupPath, args, {
    reject: false,
    stdout: "pipe",
    stderr: "pipe",
  });
}

async function checkGitStatus(targetFile: string) {
  const { stdout, exitCode } = await execa(
    `git`,
    ["diff", "--color", "--exit-code", targetFile],
    {
      reject: false,
    }
  );
  if (exitCode === 0) {
    return { state: "clean" };
  }
  return { state: "dirty", diff: stdout };
}
