#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const rootDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const distDir = path.join(rootDir, "dist");
const rootPackagePath = path.join(rootDir, "package.json");
const rootPackage = JSON.parse(fs.readFileSync(rootPackagePath, "utf8"));
const version = rootPackage.version;
const tag = version.startsWith("v") ? version : `v${version}`;

const packages = [
  {
    packageName: "@happytoolin/hni-darwin-arm64",
    directory: "npm/platforms/hni-darwin-arm64",
    binaryTargetPath: "bin/hni",
    sourceAsset: `hni-${tag}-aarch64-apple-darwin`,
  },
  {
    packageName: "@happytoolin/hni-darwin-x64",
    directory: "npm/platforms/hni-darwin-x64",
    binaryTargetPath: "bin/hni",
    sourceAsset: `hni-${tag}-x86_64-apple-darwin`,
  },
  {
    packageName: "@happytoolin/hni-linux-arm64-musl",
    directory: "npm/platforms/hni-linux-arm64-musl",
    binaryTargetPath: "bin/hni",
    sourceAsset: `hni-${tag}-aarch64-unknown-linux-musl`,
  },
  {
    packageName: "@happytoolin/hni-linux-x64-musl",
    directory: "npm/platforms/hni-linux-x64-musl",
    binaryTargetPath: "bin/hni",
    sourceAsset: `hni-${tag}-x86_64-unknown-linux-musl`,
  },
  {
    packageName: "@happytoolin/hni-win32-arm64-msvc",
    directory: "npm/platforms/hni-win32-arm64-msvc",
    binaryTargetPath: "bin/hni.exe",
    sourceAsset: `hni-${tag}-aarch64-pc-windows-msvc.exe`,
  },
  {
    packageName: "@happytoolin/hni-win32-x64-msvc",
    directory: "npm/platforms/hni-win32-x64-msvc",
    binaryTargetPath: "bin/hni.exe",
    sourceAsset: `hni-${tag}-x86_64-pc-windows-msvc.exe`,
  },
];

validateOptionalDependencyVersions(rootPackage, packages, version);

for (const pkg of packages) {
  const packageDir = path.join(rootDir, pkg.directory);
  const packageJsonPath = path.join(packageDir, "package.json");
  const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, "utf8"));

  packageJson.version = version;
  fs.writeFileSync(packageJsonPath, `${JSON.stringify(packageJson, null, 2)}\n`);

  const sourceBinaryPath = path.join(distDir, pkg.sourceAsset);
  if (!fs.existsSync(sourceBinaryPath)) {
    throw new Error(`missing dist asset: ${pkg.sourceAsset}`);
  }

  const targetBinaryPath = path.join(packageDir, pkg.binaryTargetPath);
  fs.mkdirSync(path.dirname(targetBinaryPath), { recursive: true });
  fs.copyFileSync(sourceBinaryPath, targetBinaryPath);
  if (!targetBinaryPath.endsWith(".exe")) {
    fs.chmodSync(targetBinaryPath, 0o755);
  }

  fs.copyFileSync(path.join(rootDir, "LICENSE"), path.join(packageDir, "LICENSE"));
  console.log(`[hni] prepared ${pkg.packageName}@${version}`);
}

function validateOptionalDependencyVersions(rootPkg, platformPkgs, expectedVersion) {
  const optional = rootPkg.optionalDependencies ?? {};
  for (const pkg of platformPkgs) {
    const declaredVersion = optional[pkg.packageName];
    if (declaredVersion !== expectedVersion) {
      throw new Error(
        `optionalDependencies.${pkg.packageName} must match root version (${expectedVersion}), found ${declaredVersion ?? "<missing>"}`,
      );
    }
  }
}
