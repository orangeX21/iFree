let body = $response.body;

try {
    let data = JSON.parse(body);
    if (data && data.data && data.data.current_device) {
        data.data.current_device.trial_status = 1;
        data.data.member_status = 1;
        data.data.current_device.trial_end_at = 1734875974;
        $done({ body: JSON.stringify(data) });
    }
} catch (e) {
    console.error("修改响应时出错", e);
    $done({});
}
