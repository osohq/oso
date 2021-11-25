import { resolve } from 'path';

import { runTests } from '@vscode/test-electron'; // eslint-disable-line node/no-unpublished-import
import { minVersion } from 'semver'; // eslint-disable-line node/no-unpublished-import

import { engines } from '../package.json';

void (async function () {
  try {
    // Fetch the semver constraint from the 'engines' field in the extension's
    // package.json and test against the minimum satisfiable version.
    const minSupportedVSCodeVersion = minVersion(engines.vscode).toString();

    const extensionDevelopmentPath = resolve(__dirname, '../../..');
    const extensionTestsPath = resolve(__dirname, './suite');
    const workspace = resolve(
      __dirname,
      '../../test-fixtures/workspace/test.code-workspace'
    );

    await runTests({
      version: minSupportedVSCodeVersion,
      extensionDevelopmentPath,
      extensionTestsPath,
      launchArgs: [workspace, '--disable-extensions', '--disable-telemetry'],
    });
  } catch (e) {
    console.error(e);
    process.exitCode = 1;
  }
})();
