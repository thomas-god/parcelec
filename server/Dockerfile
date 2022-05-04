FROM node:16-alpine

WORKDIR /usr/src/app

ARG NODE_ENV=development
ENV NODE_ENV=${NODE_ENV}

COPY package*.json /usr/src/app

RUN npm ci
RUN echo $NODE_ENV

COPY . /usr/src/app

RUN npx tsc

CMD npm run start:${NODE_ENV}
