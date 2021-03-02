/**
 * Define the app wide logging object.
 */

import winston from 'winston';
const { combine, timestamp, printf } = winston.format;

const my_format = printf(
  ({ level, message, timestamp, session_id, user_id, phase_no }) => {
    return `${timestamp} ${level}: SID=${
      session_id !== undefined ? session_id : ''
    } - phase=${phase_no !== undefined ? phase_no : -1}- UID=${
      user_id !== undefined ? user_id : ''
    } - ${message}`;
  }
);

const logger = winston.createLogger({
  level: 'info',
  format: combine(timestamp(), my_format),
  transports: [new winston.transports.Console()],
});

export default logger;
