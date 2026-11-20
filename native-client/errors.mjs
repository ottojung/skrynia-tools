export class NativeRejected extends Error {
  constructor(code, detail) {
    super('Skrynia native request rejected with code ' + code);
    this.name = 'NativeRejected'; this.code = code; this.detail = detail;
  }
}
export class NativeOutcomeUnknown extends Error {
  constructor(reason, cause) {
    super('Native mutation outcome unknown: ' + reason, {cause});
    this.name = 'NativeOutcomeUnknown'; this.outcomeUnknown = true;
  }
}
export class NativeTransportError extends Error {
  constructor(reason, cause) {
    super('Native read failed: ' + reason, {cause});
    this.name = 'NativeTransportError';
  }
}
