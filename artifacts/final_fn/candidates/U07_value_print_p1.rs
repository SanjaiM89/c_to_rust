// unit: U07_value_print_p1
fn format_e(d: f64, prec: usize) -> String {
    let s = match prec {
        14 => format!("{:.14e}", d),
        _ => format!("{:.16e}", d),
    };
    if let Some((mant, exp)) = s.split_once('e') {
        let mant_trimmed = mant.trim_end_matches('0').trim_end_matches('.');
        if let Ok(exp_num) = exp.parse::<i32>() {
            if exp_num < 0 {
                return format!("{}e-{:02}", mant_trimmed, exp_num.abs());
            } else {
                return format!("{}e+{:02}", mant_trimmed, exp_num);
            }
        }
    }
    s
}

fn format_g(d: f64) -> String {
    let abs_d = d.abs();
    if abs_d != 0.0 && (abs_d < 1e-4 || abs_d >= 1e15) {
        let s15 = format_e(d, 14);
        if let Ok(rec) = s15.parse::<f64>() {
            let max_val = d.abs().max(rec.abs());
            if (d - rec).abs() <= max_val * f64::EPSILON {
                return s15;
            }
        }
        format_e(d, 16)
    } else {
        let s15 = format!("{:.14}", d);
        let s15_trimmed = s15.trim_end_matches('0').trim_end_matches('.');
        if let Ok(rec15) = s15_trimmed.parse::<f64>() {
            let max_val = d.abs().max(rec15.abs());
            if (d - rec15).abs() <= max_val * f64::EPSILON {
                return s15_trimmed.to_string();
            }
        }
        let s17 = format!("{:.16}", d);
        s17.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

pub fn print_number(item: &Node, output_buffer: &mut PrintBuffer) -> bool {
    let d = item.valuedouble;
    let s = if d.is_nan() || d.is_infinite() {
        "null".to_string()
    } else if d == item.valueint as f64 {
        format!("{}", item.valueint)
    } else {
        format_g(d)
    };

    let bytes = s.as_bytes();
    let length = bytes.len();
    let offset = match ensure(output_buffer, length + 1) {
        Some(off) => off,
        None => return false,
    };

    output_buffer.buffer[offset..offset + length].copy_from_slice(bytes);
    output_buffer.buffer[offset + length] = 0;
    output_buffer.offset += length;
    true
}
