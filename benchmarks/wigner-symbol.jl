using WignerSymbols


function compute_all(max_angular)
    for l1 = 0:max_angular
        for l2 = 0:max_angular
            for l3 = 0:max_angular
                for m1 = -l1:l1
                    for m2 = -l2:l2
                        for m3 = -l3:l3
                            wigner3j(Float64, l1, l2, l3, m1, m2, m3)
                        end
                    end
                end
            end
        end
    end
end

function compute_large(j1, j2, j3)
    for m1 = -10:10
        for m2 = -10:10
            for m3 = -10:10
                wigner3j(Float64, j1, j2, j3, m1, m2, m3)
            end
        end
    end
end


for max_angular in [4, 8, 12, 16, 20]
    println("max_angular = $max_angular")
    # warmup & compile
    compute_all(max_angular)
    empty!(WignerSymbols.Wigner3j)
    GC.gc()

    @time compute_all(max_angular)
end


println("j = (300, 100, 250)")
# warmup & compile
compute_large(300, 100, 250)
empty!(WignerSymbols.Wigner3j)
GC.gc()

@time compute_large(300, 100, 250)
