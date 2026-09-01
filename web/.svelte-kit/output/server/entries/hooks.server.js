import { sequence } from "@sveltejs/kit/hooks";
const handle = sequence(async ({ event, resolve }) => {
  return resolve(event);
});
export {
  handle
};
