import java.util.*;
import org.json.JSONObject;
import org.json.JSONException;
import org.json.JSONArray;

public class Query implements Enumeration<HashMap<String, Object>> {
    private HashMap<String, Object> next;
    private Ffi.QueryPtr queryPtr;
    private Polar polar;

    /**
     * Construct a new Query object.
     *
     * @param queryPtr Pointer to the FFI query instance.
     */
    public Query(Ffi.QueryPtr queryPtr, Polar polar) throws Exceptions.OsoException {
        this.queryPtr = queryPtr;
        this.polar = polar;
        next = nextResult();
    }

    @Override
    public boolean hasMoreElements() {
        return next != null;
    }

    @Override
    public HashMap<String, Object> nextElement() {
        HashMap<String, Object> ret = next;
        try {
            next = nextResult();
        } catch (Exception e) {
            throw new NoSuchElementException("Caused by: e.toString()");
        }
        return ret;
    }

    /**
     * Get all query results
     *
     * @return List of all query results (binding sets)
     */
    public List<HashMap<String, Object>> results() {
        List<HashMap<String, Object>> results = Collections.list(this);
        return results;
    }

    /**
     * Helper for `ExternalCall` query events
     *
     * @param attrName
     * @param jArgs
     * @param instanceId
     * @param callId
     * @throws Exceptions.OsoException
     */
    private void handleCall(String attrName, JSONArray jArgs, long instanceId, long callId)
            throws Exceptions.OsoException {
        List<Object> args = polar.polarListToJava(jArgs);
        polar.registerCall(attrName, args, callId, instanceId);
        String result;
        try {
            result = polar.nextCallResult(callId).toString();
        } catch (NoSuchElementException e) {
            result = null;
        }
        queryPtr.polarCallResult(callId, result);
    }

    /**
     * Generate the next Query result
     *
     * @return
     * @throws Exceptions.OsoException
     */
    private HashMap<String, Object> nextResult() throws Exceptions.OsoException {
        while (true) {
            String eventStr = queryPtr.polarNextQueryEvent();
            String kind;
            JSONObject data;
            try {
                JSONObject event = new JSONObject(eventStr);
                kind = event.keys().next();
                data = event.getJSONObject(kind);
            } catch (JSONException e) {
                // TODO: this sucks, we should have a consistent serialization format
                kind = eventStr.replace("\"", "");
                data = null;
            }

            switch (kind) {
                case "Done":
                    return null;
                case "Result":
                    return polar.polarDictToJava(data.getJSONObject("bindings"));
                case "MakeExternal":
                    Long id = data.getLong("instance_id");
                    if (polar.hasInstance(id)) {
                        throw new Exceptions.DuplicateInstanceRegistrationError(id);
                    }
                    String clsName = data.getJSONObject("instance").getString("tag");
                    JSONObject jFields = data.getJSONObject("instance").getJSONObject("fields").getJSONObject("fields");
                    polar.makeInstance(clsName, polar.polarDictToJava(jFields), id);
                    break;
                case "ExternalCall":
                    long callId = data.getLong("call_id");
                    long instanceId = data.getLong("instance_id");
                    String attrName = data.getString("attribute");
                    JSONArray jArgs = data.getJSONArray("args");
                    handleCall(attrName, jArgs, instanceId, callId);
                    break;
                case "ExternalIsa":
                    instanceId = data.getLong("instance_id");
                    callId = data.getLong("call_id");
                    String classTag = data.getString("class_tag");
                    int answer = polar.isa(instanceId, classTag) ? 1 : 0;
                    queryPtr.polarQuestionResult(callId, answer);
                    break;
                case "ExternalIsSubSpecializer":
                    instanceId = data.getLong("instance_id");
                    callId = data.getLong("call_id");
                    String leftTag = data.getString("left_class_tag");
                    String rightTag = data.getString("right_class_tag");
                    answer = polar.subspecializer(instanceId, leftTag, rightTag) ? 1 : 0;
                    queryPtr.polarQuestionResult(callId, answer);
                    break;
                default:
                    throw new Exceptions.PolarRuntimeException("Unhandled event type: " + kind);
            }
        }

    }

}