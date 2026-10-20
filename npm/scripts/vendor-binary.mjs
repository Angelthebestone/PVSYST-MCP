// Stages the release artifact into the package before publishing: the compiled
// server plus the license files, so the tarball is self-contained (Apache-2.0
// sections 4(a) and 4(d) require the license and the NOTICE to travel with it).
import { copyFileSync, existsSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const pkg = join(here, "..");
const root = join(pkg, "..");
const exe = join(root, "target", "release", "pvsyst-mcp.exe");

if (!existsSync(exe)) {
  process.stderr.write(
    `vendor-binary: ${exe} not found. Run "cargo build --release" in ${root} first.\n`,
  );
  process.exit(1);
}

mkdirSync(join(pkg, "vendor"), { recursive: true });
copyFileSync(exe, join(pkg, "vendor", "pvsyst-mcp.exe"));
for (const name of ["LICENSE", "NOTICE"]) {
  copyFileSync(join(root, name), join(pkg, name));
}
process.stdout.write("vendor-binary: staged pvsyst-mcp.exe, LICENSE and NOTICE\n");
