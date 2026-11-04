let charset = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c", "d", "e", "f"];
let generate_string = |len| {
    let data = [];
    for i in 0..len {
        data.push(charset[rand(charset.len)]);
    }
    return concat(data);
};

fn swap(i, j) {
    let t = this[i];
    this[i] = this[j];
    this[j] = t;
}

fn partition(lo, hi) {
    let pivot_idx = (lo + hi) / 2;
    let pivot = this[pivot_idx];
    this.swap(pivot_idx, hi);
    let j = lo;
    for i in lo..hi {
        if this[i] < pivot {
            this.swap(i, j);
            j = j + 1;
        }
    }
    this.swap(j, hi);
    return j;
}

fn quicksort(lo, hi) {
    // Each triple stores lo, hi, and the previous stack entry (0 = empty).
    let ranges = [lo, hi, 0];
    let top = 3;
    while top > 0 {
        lo = ranges[top - 3];
        hi = ranges[top - 2];
        top = ranges[top - 1];
        while lo < hi {
            let p = this.partition(lo, hi);
            if p + 1 < hi {
                ranges.push(p + 1);
                ranges.push(hi);
                ranges.push(top);
                top = ranges.len;
            }
            hi = p - 1;
        }
    }
}

let array = [];
for i in 0..10000 {
    array.push(RustData_new(generate_string.call(8 + rand(16))));
}
array.quicksort(0, array.len - 1);
return array;
