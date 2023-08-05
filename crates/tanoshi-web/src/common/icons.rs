use dominator::{svg, Dom};

pub fn chevron_left() -> Dom {
    svg!("svg", {
        .attr("xmlns", "http://www.w3.org/2000/svg")
        .attr("fill", "none")
        .attr("viewBox", "0 0 24 24")
        .attr("stroke", "currentColor")
        .class("icon")
        .children(&mut [
            svg!("path", {
                .attr("stroke-linecap", "round")
                .attr("stroke-linejoin", "round")
                .attr("stroke-width", "2")
                .attr("d", "M15 19l-7-7 7-7")
            })
        ])
    })
}

pub fn refresh() -> Dom {
    svg!("svg", {
        .attr("xmlns", "http://www.w3.org/2000/svg")
        .attr("fill", "none")
        .attr("viewBox", "0 0 24 24")
        .attr("stroke", "currentColor")
        .class("icon")
        .children(&mut [
            svg!("path", {
                .attr("stroke-linecap", "round")
                .attr("stroke-linejoin", "round")
                .attr("stroke-width", "2")
                .attr("d", "M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15")
            })
        ])
    })
}

pub fn external_link() -> Dom {
    svg!("svg", {
        .attr("xmlns", "http://www.w3.org/2000/svg")
        .attr("fill", "currentColor")
        .attr("viewBox", "0 0 20 20")
        .class("icon")
        .children(&mut [
            svg!("path", {
                .attr("d", "M11 3a1 1 0 100 2h2.586l-6.293 6.293a1 1 0 101.414 1.414L15 6.414V9a1 1 0 102 0V4a1 1 0 00-1-1h-5z")
            }),
            svg!("path", {
                .attr("d", "M5 5a2 2 0 00-2 2v8a2 2 0 002 2h8a2 2 0 002-2v-3a1 1 0 10-2 0v3H5V7h3a1 1 0 000-2H5z")
            })
        ])
    })
}

pub fn checkmark() -> Dom {
    svg!("svg", {
        .attr("xmlns", "http://www.w3.org/2000/svg")
        .attr("fill", "currentColor")
        .attr("viewBox", "0 0 20 20")
        .class("icon-sm")
        .children(&mut [
            svg!("path", {
                .attr("fill-rule", "evenodd")
                .attr("d", "M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z")
                .attr("clip-rule", "evenodd")
            })
        ])
    })
}

pub fn crossmark() -> Dom {
    svg!("svg", {
        .attr("xmlns", "http://www.w3.org/2000/svg")
        .attr("fill", "currentColor")
        .attr("viewBox", "0 0 20 20")
        .class("icon-sm")
        .children(&mut [
            svg!("path", {
                .attr("fill-rule", "evenodd")
                .attr("d", "M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z")
                .attr("clip-rule", "evenodd")
            })
        ])
    })
}
