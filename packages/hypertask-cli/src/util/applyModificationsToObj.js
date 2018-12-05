import * as R from "ramda";
import parseDateTimeShortcut, {
	parseRecur,
} from "../util/parseDateTimeShortcut";
const dateTimeProps = new Set(["due", "wait", "sleep", "snooze"]);

const applyModificationsToObj = modifications => async task => {
	for (const { prop, plus, minus } of modifications) {
		if (prop) {
			const [key] = R.keys(prop);
			const [value] = R.values(prop);

			if (key === "description" && value.length === 0) {
				continue;
			}

			try {
				if (value === null) {
					await task[`${key}Set`](undefined);
				} else {
					console.log({ key, value });
					if (dateTimeProps.has(key)) {
						await task[`${key}Set`](parseDateTimeShortcut(value));
					} else if (key === "recur") {
						await task[`${key}Set`](parseRecur(value));
					} else {
						await task[`${key}Set`](value);
					}
				}
			} catch (e) {
				console.error(`Error, ${key} doesn't seem to be a valid prop!`);
			}
		}

		if (plus) {
			await task.tagsAdd(plus);
		}

		if (minus) {
			await task.tagsRemove(minus);
		}
	}
};

export default applyModificationsToObj;
