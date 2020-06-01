// TODO: set logger
// TODO: test without TRAC_ADMIN
// TODO: test multi-root project
// TODO: concat tests: create -> update

fn config() -> tracpost::Config {
    tracpost::Config {
        trac_user: "wsh".to_string(),
        trac_pass: "1".to_string(),
        url_rpc: Some("http://localhost:8000/login/rpc".to_string()),
        update_comment: None,
        test_mode: true,
    }
}

// TOOD: test only if trac available
#[test]
fn test_post_create() {
    // create (url: (empty))
    let txt = textwrap::dedent(
        &r"
            {{{
            #!comment
            url_rpc: http://localhost:8000/login/rpc
            # url:
            
            summary: hi there
            
            # component: component1
            milestone: milestone1
            priority: blocker
            # resolution: fixed
            severity:
            # status: accepted
            type: defect
            
            foo: bar
            # owner:
            # description -> warning?
            }}}
            
            test test
            test
            "[1..],
    );
    let _tmp = tracpost::post(&config(), &txt);
    // TODO
    let _breakpoint = 1;
}

#[test]
// #[ignore]
fn test_post_upate() {
    // update (url: (ticket url))
    let txt = textwrap::dedent(
        &r"
            {{{
            #!comment
            url_rpc: http://localhost:8000/login/rpc
            id: 1
            # url: http://localhost:8000/ticket/1  # TODO
            
            summary: hi there
            
            component:
            milestone: milestone9999
            priority: blocker
            resolution: fixed
            severity:
            # status: accepted
            type: defect
            
            # foo: bar
            }}}
            
            test test
            test
            "[1..],
    );
    let _tmp = tracpost::post(&config(), &txt);
    let txt = textwrap::dedent(
        &r"
            {{{
            #!comment
            url_rpc: http://localhost:8000/login/rpc
            id: 1
            
            summary: hi there
            
            component: component1    # +
            # milestone:             # unchanged
            priority: blocker
            # resolution: fixed
            severity:
            # status: accepted
            type: defect
            }}}
            
            test test
            test
            "[1..],
    );
    let _tmp = tracpost::post(&config(), &txt);
    let _breakpoint = 1;
}

#[test]
// #[ignore]
fn test_headers() {
    // TODO: assert error

    // invalid start
    let txt = textwrap::dedent(
        &r"
            invalid start
            {{{
            #!comment
            }}}"[1..],
    );
    let _tmp = tracpost::post(&config(), &txt);

    // missing colong
    let txt = textwrap::dedent(
        &r"
            {{{
            #!comment
            missing colon
            }}}"[1..],
    );
    let _tmp = tracpost::post(&config(), &txt);

    // duplicate
    let txt = textwrap::dedent(
        &r"
            {{{
            #!comment
            key1: val1
            key1: val2
            }}}"[1..],
    );
    let _tmp = tracpost::post(&config(), &txt);

    // no url
    let txt = textwrap::dedent(
        &r"
            {{{
            #!comment
            }}}"[1..],
    );
    let _tmp = tracpost::post(&config(), &txt);

    let _breakpoint = 1;
}
