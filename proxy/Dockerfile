FROM node:14.15

WORKDIR /app
COPY package.json package.json
COPY yarn.lock yarn.lock
RUN yarn install
COPY tsconfig.json .
COPY src/ src/

ENTRYPOINT yarn start
