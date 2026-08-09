const positiveIntegerPattern = /^[1-9]\d*$/;

/**
 * Keep local validation responsive by default while leaving CI concurrency
 * unchanged. Set the named environment variable to explicitly override it.
 */
export function localTestWorkerCap(variableName, defaultValue) {
  const configured = process.env[variableName];
  if (configured === undefined || configured === "") {
    return process.env.CI ? undefined : defaultValue;
  }

  if (!positiveIntegerPattern.test(configured)) {
    throw new Error(`${variableName} must be a positive integer`);
  }

  const workerCount = Number(configured);
  if (!Number.isSafeInteger(workerCount)) {
    throw new Error(`${variableName} must be a positive integer`);
  }

  return workerCount;
}
