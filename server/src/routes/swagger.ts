import { promises as fs } from 'fs';
import swaggerJSDoc from 'swagger-jsdoc';
import jsyaml from 'js-yaml';
import { resolve } from 'path'

const options = {
  swaggerDefinition: {
    openapi: '3.0.0',
    info: {
      title: 'Parcelec API',
      version: '1.0.0'
    }
  },
  apis: ['./src/routes/**/*Controller.ts']
}

export const swaggerDefinition = swaggerJSDoc(options);

if (require.main == module) {
  fs.writeFile(resolve(__dirname, '../../openapi.yaml'), jsyaml.dump(swaggerDefinition))
}