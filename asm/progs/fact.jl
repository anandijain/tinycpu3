function fact(n)
    if n > 1
        n*factorial(n-1)
    else 
        1
    end
end

n = 4
check n>1 true
n * factorial 