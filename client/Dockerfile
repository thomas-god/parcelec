FROM node:22 AS build

# Set the working directory
WORKDIR /app

# Copy the package.json and package-lock.json
COPY package*.json ./

# Install the dependencies
RUN npm ci

# Copy the rest of the application
COPY . .

# Build the application for production
ENV NODE_ENV=production
RUN npm run build

# Use the official Nginx image
FROM nginx:alpine

# Copy the build output to replace the default nginx contents
COPY --from=build /app/public /usr/share/nginx/html
COPY --from=build /app/dist /usr/share/nginx/html

# Expose the port
EXPOSE 80

# Start Nginx
CMD ["nginx", "-g", "daemon off;"]
