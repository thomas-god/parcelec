export class UserDoesNotExistError extends Error {
  constructor(message) {
    super(message);
    this.name = 'UserDoesNotExistError';
  }
}
