FROM node:12

RUN apt install curl wget g++ make python

# Install wait-for-it
RUN curl -o /root/wait-for-it.sh  https://raw.githubusercontent.com/vishnubob/wait-for-it/master/wait-for-it.sh
RUN chmod +x /root/wait-for-it.sh

WORKDIR /usr/app

COPY package-lock.json /usr/app/
COPY package.json /usr/app/
RUN npm install
RUN npm install mocha-jenkins-reporter

COPY . /usr/app/

ENV TZ=Europe/Paris
ENV NODE_ENV=test

# Wait for DB and run test
CMD /root/wait-for-it.sh -t 90 db:5432 \
   && node node_modules/mocha/bin/mocha -r ts-node/register e2e/**/*.test.e2e.ts --recursive --exit --colors