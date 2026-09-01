import "./client.js";
import { g as getContext } from "./index.js";
import "clsx";
function context() {
  return getContext("__request__");
}
const page$1 = {
  get status() {
    return context().page.status;
  },
  get url() {
    return context().page.url;
  }
};
const page = page$1;
export {
  page as p
};
