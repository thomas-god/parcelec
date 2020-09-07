FROM node:12-alpine

WORKDIR /usr/src/app

ARG NODE_ENV=development
ENV NODE_ENV=${NODE_ENV}

COPY package.json ./package.json
COPY package-lock.json ./package-lock.json

RUN npm install
RUN echo $NODE_ENV

COPY . .

RUN npx tsc

CMD npm run start:${NODE_ENV}