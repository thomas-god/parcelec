export interface Phase {
  sessionId: string;
  phaseNo: number;
  status: 'open' | 'closed';
  startTime: Date;
  clearingTime: Date;
  planningTime: Date;
  bidsAllowed: boolean;
  clearingAvailable: boolean;
  planningsAllowed: boolean;
  resultsAvailable: boolean;
}
