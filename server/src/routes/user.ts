import express from "express";
const router = express.Router();

const usernames: string[] = ["Toto", "Tutu", "Tata", "Titi"];

router.get("/list", (req, res) => {
  res.json(usernames);
});

router.put("/new", (req, res) => {
  console.log(req.body);
  if (req.body.username) {
    if (!usernames.includes(req.body.username)) {
      usernames.push(req.body.username);
      res.end();
    } else {
      res.status(400).end("Error, username already exist");
    }
  }
});

export function checkUsernameExists(username: string): boolean {
  return usernames.includes(username);
}

export default router;
