require 'ffi'
require 'json'
require_relative 'polar_lib'


def check_result(result)
    if result == 0 || result.null?
        # TODO: raise error
        puts 'ERROR!'
    result
end

def unstringify(s)
    check_result(string)
    JSON.parse(string)
ensure
    if !string.null?
        PolarLib.string_free(string)
    end
end

class Polar
    attr_reader :polar

    def initialize()
        puts 'init'
        @polar = PolarLib.polar_new()
    end

    # Load a Polar string, checking that all inline queries succeed.
    def load_str(str)
        check_result(PolarLib.load_str(str))
    end

    def query_str(str)
        query = check_result(PolarLib.polar_new_query(self.polar, str))
        # yield from self.run_query(query)
        # TODO: figure out what to do here
    end

    def run_query(q)
        loop do
            event_s = PolarLib.polar_query(self.polar, q)
            event = unstringify(event_s)
            if event == 'Done'
                break
            kind = [*event][0]
            data = event[kind]

            # if kind == 'MakeExternal':
            #     self.__handle_make_external(data)
            # if kind == 'ExternalCall':
            #     self.__handle_external_call(query, data)
            # if kind == 'ExternalIsa':
            #     self.__handle_external_isa(query, data)
            # if kind == 'ExternalIsSubSpecializer':
            #     self.__handle_external_is_subspecializer(query, data)
            # if kind == 'Debug':
            #     self.__handle_debug(query, data)
            if kind == 'Result':
                # yield data['bindings'].map{|k,v| [k, self.to_ruby(v)]}.to_h
                # TODO: figure out what to do here
        end

    ensure
        PolarLib.query_free(q)
    end

    def to_ruby(value)
        value = v['value']
        tag = [*value][0]
        case tag
        when 'Integer', 'String', 'Boolean'
            value[tag]
        when 'List'
            value[tag].map {|e| self.to_ruby(e)}
        when 'Dictionary'
            value[tag]['fields'].map {|k,v| [k, self.to_ruby(v)]}.to_h
        # elsif tag == 'ExternalInstance':
        #     return self.__get_instance(value[tag]['instance_id'])
        # elif tag == 'InstanceLiteral':
        #     # TODO(gj): Should InstanceLiterals ever be making it to Python?
        #     # convert instance literals to external instances
        #     cls_name = value[tag]['tag']
        #     fields = value[tag]['fields']['fields']
        #     return self.__make_external_instance(cls_name, fields)
        # elif tag == 'Call':
        #     return Predicate(
        #         name=value[tag]['name'],
        #         args=[self._to_python(v) for v in value[tag]['args']],
        #     )
        when 'Symbol'
            # TODO: raise error here
            puts 'ERROR!'
            # raise PolarRuntimeException(
            #     f'variable: {value} is unbound. make sure the value is set before using it in a method call'
            # )
        else
            # TODO: raise error here
            puts 'ERROR!'
            # raise PolarRuntimeException(f'cannot convert: {value} to Python')
        end



ensure
    puts 'ensure'
end

# test FFI
p = Polar.new()
sleep 2