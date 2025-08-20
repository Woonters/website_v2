#[macro_export]
macro_rules! slide_in_and_out {
    ($t:expr,$c:expr) => {{
        fx::sequence(&[
            fx::prolong_start(
                $t,
                fx::slide_out(
                    Motion::DownToUp,
                    10,
                    1,
                    $c,
                    EffectTimer::from_ms(500, Interpolation::Linear),
                ),
            ),
            fx::slide_in(
                Motion::DownToUp,
                10,
                1,
                $c,
                EffectTimer::from_ms(500, Interpolation::Linear),
            ),
        ])
    }};
}

#[macro_export]
macro_rules! slide_in_and_out_disp {
    ($t:expr,$c:expr,$s:expr,$e_start:expr) => {{
        fx::sequence(&[
            fx::prolong_start(
                $t,
                fx::slide_out(
                    Motion::DownToUp,
                    10,
                    1,
                    $c,
                    EffectTimer::from_ms(500, Interpolation::Linear),
                ),
            ),
            fx::dispatch_event($s, $e_start),
            fx::slide_in(
                Motion::DownToUp,
                10,
                1,
                $c,
                EffectTimer::from_ms(500, Interpolation::Linear),
            ),
            // fx::dispatch_event($s, $e_end),
        ])
    }};
}

#[macro_export]
macro_rules! animate {
    ($effect:expr,$frame:expr,$position:expr,$duration:expr) => {{
        if $effect.running() {
            $frame.render_effect(&mut $effect, $position, Duration::from_millis($duration))
        }
    }};
    (( $(( $effect:expr, $position:expr )),*),$frame:expr,$duration:expr) => {$({
        if $effect.running() {
            $frame.render_effect(&mut $effect, $position, Duration::from_millis($duration));
        }
    })*};
}
