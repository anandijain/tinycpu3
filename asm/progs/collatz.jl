function collatz_steps(n)
    n_steps = 0
    steps = [n]
    while n != 1
        n_steps += 1
        if n % 2 == 0
            n = n / 2
        else
            n = 3 * n + 1
        end
        push!(steps, n)
    end
    n_steps, steps
end

collatz_steps(7)