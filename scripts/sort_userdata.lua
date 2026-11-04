local charset = { "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c", "d", "e", "f" }
local function generate_string(len)
	local data = table.create(len)
	for _ = 1, len do
		table.insert(data, charset[rand(#charset) + 1])
	end
	return table.concat(data)
end

local function partition(arr, lo, hi)
	local pivot_idx = math.floor((lo + hi) / 2)
	local pivot = arr[pivot_idx]
	arr[pivot_idx], arr[hi] = arr[hi], arr[pivot_idx]
	local j = lo
	for i = lo, hi - 1 do
		if arr[i] < pivot then
			arr[i], arr[j] = arr[j], arr[i]
			j = j + 1
		end
	end
	arr[j], arr[hi] = arr[hi], arr[j]
	return j
end

local function quicksort(arr, lo, hi)
	-- Each triple stores lo, hi, and the previous stack entry (0 = empty).
	local ranges = {lo, hi, 0}
	local top = 3
	while top > 0 do
		lo = ranges[top - 2]
		hi = ranges[top - 1]
		top = ranges[top]
		while lo < hi do
			local p = partition(arr, lo, hi)
			if p + 1 < hi then
				local size = #ranges
				ranges[size + 1], ranges[size + 2], ranges[size + 3] = p + 1, hi, top
				top = size + 3
			end
			hi = p - 1
		end
	end
end

function bench()
	local array = {}
	for _ = 1, 10000 do
		table.insert(array, RustData.new(generate_string(8 + rand(16))))
	end
	quicksort(array, 1, #array)
	return array
end
