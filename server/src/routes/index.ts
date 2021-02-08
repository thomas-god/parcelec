import { Application } from 'express';
import { container, Dependencies } from '../di.context'
import { BidsController } from './bids/BidsController'

export default function (app: Application) {
	(container.resolve("BidsController") as Dependencies["BidsController"]).init(app)
}