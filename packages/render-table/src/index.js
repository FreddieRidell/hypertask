import * as R from "ramda";

const renderTable = R.curry((columns, data) => {
	const columnWidths = data.reduce(
		(widths, task) => ({
			...widths,
			...R.fromPairs(
				R.toPairs(task).map(([key, value]) => [
					key,
					Math.max(("" + value).length, widths[key] || 0),
				]),
			),
		}),
		R.fromPairs(columns.map(key => [key, key.length])),
	);

	const lines = [
		"\u001b[4m" +
			columns
				.map(
					R.pipe(
						col => col.padEnd(columnWidths[col]),
						R.split(""),
						R.over(R.lensIndex(0), R.toUpper),
						R.join(""),
					),
				)
				.join(" ") +
			"\u001b[0m",
	];

	for (const datum of data) {
		const line = [];
		for (const col of columns) {
			line.push(("" + (datum[col] || "")).padEnd(columnWidths[col]));
		}

		lines.push(line.join(" "));
	}

	return lines.join("\n");
});

export default renderTable;