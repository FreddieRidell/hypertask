import createParseCliArgs from "../src";

describe("query", () => {
	let parseCliArgs;

	beforeEach(() => {
		parseCliArgs = createParseCliArgs(["add", "modify", "delete"]);
	});

	describe("plus tags", () => {
		it("parses one", () => {
			const response = parseCliArgs(["+foo"]);

			expect(response).toMatchObject({
				query: [{ plusTag: "foo" }],
			});
		});

		it("parses many", () => {
			const response = parseCliArgs(["+foo", "+bar", "+baz"]);

			expect(response).toMatchObject({
				query: [
					{ plusTag: "foo" },
					{ plusTag: "bar" },
					{ plusTag: "baz" },
				],
			});
		});
	});

	describe("minus tags", () => {
		it("parses one", () => {
			const response = parseCliArgs(["-foo"]);

			expect(response).toMatchObject({
				query: [{ minusTag: "foo" }],
			});
		});

		it("parses many", () => {
			const response = parseCliArgs(["-foo", "-bar", "-baz"]);

			expect(response).toMatchObject({
				query: [
					{ minusTag: "foo" },
					{ minusTag: "bar" },
					{ minusTag: "baz" },
				],
			});
		});
	});

	describe("strings", () => {
		it("parses one", () => {
			const response = parseCliArgs(["foo"]);

			expect(response).toMatchObject({
				query: [{ string: "foo" }],
			});
		});

		it("parses many", () => {
			const response = parseCliArgs(["foo", "bar", "baz"]);

			expect(response).toMatchObject({
				query: [
					{ string: "foo" },
					{ string: "bar" },
					{ string: "baz" },
				],
			});
		});
	});

	describe("props", () => {
		it("parses one", () => {
			const response = parseCliArgs(["foo:bar"]);

			expect(response).toMatchObject({
				query: [{ prop: { foo: "bar" } }],
			});
		});

		it("parses many", () => {
			const response = parseCliArgs(["foo:bar", "baz:qux", "a:b"]);

			expect(response).toMatchObject({
				query: [
					{ prop: { foo: "bar" } },
					{ prop: { baz: "qux" } },
					{ prop: { a: "b" } },
				],
			});
		});
	});

	it("parses a complex array of arguments", () => {
		const response = parseCliArgs([
			"due:now",
			"recur:2d",
			"+chores",
			"-timely",
			"do",
			"the",
			"dishes",
			"more",
			"often",
		]);

		expect(response).toMatchObject({
			query: [
				{ prop: { due: "now" } },
				{ prop: { recur: "2d" } },
				{ plusTag: "chores" },
				{ minusTag: "timely" },
				{ string: "do" },
				{ string: "the" },
				{ string: "dishes" },
				{ string: "more" },
				{ string: "often" },
			],
		});
	});
});
