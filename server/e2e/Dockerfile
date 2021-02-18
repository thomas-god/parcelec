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

# Wait for DB and run test
CMD /root/wait-for-it.sh -t 90 db:5432 \
   && npm run test