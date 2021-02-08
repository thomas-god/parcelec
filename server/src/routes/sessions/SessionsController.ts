import { Application, Request, Response } from 'express';
import { container, Dependencies } from '../../di.context'
import { uuid_regex } from '../utils';

export class SessionsController {
	private SessionsService: Dependencies["SessionsService"];

	constructor() {
		this.SessionsService = container.resolve("SessionsService");
	}

	init(app: Application): void {

	}
}