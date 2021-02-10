export class SessionDoesNotExistError extends Error {
    constructor(message) {
      super(message);
      this.name = 'SessionDoesNotExistError';
    }
  }

export class SessionIsNotRunningError extends Error {
    constructor(message) {
      super(message);
      this.name = 'SessionIsNotRunningError';
    }
  }