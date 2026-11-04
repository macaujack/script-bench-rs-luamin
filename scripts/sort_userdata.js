const charset = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c", "d", "e", "f"];
function generate_string(len) {
    let data = new Array(len);
    for (let i = 0; i < len; i++) {
        data.push(charset[rand(charset.length)]);
    }
    return data.join("");
}

function swap(arr, i, j) {
    const t = arr[i];
    arr[i] = arr[j];
    arr[j] = t;
}

function partition(arr, lo, hi) {
    let pivot_idx = Math.floor((lo + hi) / 2);
    let pivot = arr[pivot_idx];
    swap(arr, pivot_idx, hi);
    let j = lo;
    for (let i = lo; i < hi; i++) {
        if (arr[i].lt(pivot)) {
            swap(arr, i, j);
            j++;
        }
    }
    swap(arr, j, hi);
    return j;
}

function quicksort(arr, lo, hi) {
    // Each triple stores lo, hi, and the previous stack entry (0 = empty).
    let ranges = [lo, hi, 0];
    let top = 3;
    while (top > 0) {
        lo = ranges[top - 3];
        hi = ranges[top - 2];
        top = ranges[top - 1];
        while (lo < hi) {
            let p = partition(arr, lo, hi);
            if (p + 1 < hi) {
                ranges.push(p + 1, hi, top);
                top = ranges.length;
            }
            hi = p - 1;
        }
    }
}

function bench() {
    let array = [];
    for (let i = 0; i < 10000; i++) {
        array.push(new RustData(generate_string(8 + rand(16))));
    }
    quicksort(array, 0, array.length - 1);
    return array;
}
