FROM node:16

# Install various utilities
RUN apt update && apt install -y vim curl wget

# Install wait-for-it
RUN curl -o /root/wait-for-it.sh  https://raw.githubusercontent.com/oui-team/wait-for-it/master/wait-for-it.sh
RUN chmod +x /root/wait-for-it.sh

RUN mkdir -p /usr/app
WORKDIR /usr/app

COPY package-lock.json /usr/app/
COPY package.json /usr/app/
RUN npm ci

COPY . /usr/app/

ENV TZ=utc
ENV NODE_ENV=test

CMD /root/wait-for-it.sh -t 90 db:5432 \
 && npm run dev