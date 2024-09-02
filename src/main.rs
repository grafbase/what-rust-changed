fn main() {
    println!("Hello, world!");

    // TODO: Things what need to be done:
    // 1. Get the projects that have changed
    //    - Get files that have changed
    //      - If main branch this is diff from HEAD^
    //      - If PR this is merge-base between current branch & PR base branch
    //      - If not PR but not main probably just do merge-base between current branch & main
    //    - Take this and map to cargo proejcts
    // 2. Get deps that have changed
    //    - Open & read lockfile
    //    - Open & read lockfile in base commit, where-ever that is.
    //      (Possibly using github API for this, not sure)
    //    - Diff the dependencies somehow?  Not entirely sure how
    // 3. Combine the outputs from 1 & 2 somehow
    // 4. Use this to generate a list of projects that have directly changed
    // 5. Use that + a graph of deps to find projects that have been impacted
    // 6. Possibly also use that to find binary targets that have been impacted

    // Implementation plan:
    // Step 1 is trivial, unsurprising and easy to fake so skip it for now and take input
    // Step 2:
    // - Probably do need the comparison code but could skip doing git things for now?
    // Step 3-6 probably essential.

    // Testing plan
    // Ideally I want to see how effective this would be by looking at previous builds?
    // - Fetch PRs,
    // - Feed data for each PR in
    // - Need some way of measuring how much effort could be saved?
    //   Maybe # of deps changed vs the total, graphed or something?
}
