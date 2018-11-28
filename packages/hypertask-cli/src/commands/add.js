import * as R from "ramda";

import getId from "@hypercortex/easy-type-id";

import renderTable from "../util/renderTable";
import parseDateTimeShortcut from "../util/parseDateTimeShortcut";

const dateTimeProps = new Set(["due", "wait", "sleep", "snooze"]);

const add = async ({ filter, modifications, taskAll, task }) => {
	const newID = getId(16);
	const newTask = task(newID);

	for (const { prop, plus, minus } of modifications) {
		if (prop) {
			const [key] = R.keys(prop);
			const [value] = R.values(prop);
			if (dateTimeProps.has(key)) {
				await newTask[`${key}Set`](parseDateTimeShortcut(value));
			} else {
				await newTask[`${key}Set`](value);
			}
		}

		if (plus) {
			await newTask.tagsAdd(plus);
		}

		if (minus) {
			await newTask.tagsRemove(minus);
		}
	}

	const allTasks = await taskAll();

	const allObjects = await Promise.all(allTasks.map(t => t.toJsObject()));

	R.pipe(
		R.map(R.when(R.propEq("id", newID), R.assoc("textColor", 2))),
		renderTable,
	)(allObjects);
};

export default add;