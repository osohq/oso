# frozen_string_literal: true

Bar = DFH.record(:id, :is_cool, :is_still_cool) do
  def foos
    Foo.all.select { |foo| id == foo.bar_id }
  end
end

Foo = DFH.record(:id, :bar_id, :is_fooey, :numbers) do
  def bar
    Bar.all.find { |bar| bar.id == bar_id }
  end

  def logs
    Log.all.select { |log| id == log.foo_id }
  end
end

Log = DFH.record(:id, :foo_id, :data) do
  def foo
    Foo.all.find { |foo| foo.id == foo_id }
  end
end

Foo.new('something', 'hello', false, [])
Foo.new('another', 'hello', true, [1])
Foo.new('third', 'hello', true, [2])
Foo.new('fourth', 'goodbye', true, [2, 1, 0])

Bar.new('hello', true, true)
Bar.new('goodbye', false, true)
Bar.new('hershey', false, false)

Log.new('a', 'fourth', 'goodbye')
Log.new('b', 'third', 'world')
Log.new('c', 'another', 'steve')

Widget = DFH.record :id
0.upto(9).each { |id| Widget.new id }
